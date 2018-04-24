// Copyright (c) 2004-present, Facebook, Inc.
// All Rights Reserved.
//
// This software may be used and distributed according to the terms of the
// GNU General Public License version 2 or any later version.

#![deny(warnings)]
#![feature(conservative_impl_trait)]

extern crate blobrepo;
extern crate bytes;
extern crate changesets;
extern crate clap;
extern crate failure_ext;
extern crate futures;
#[macro_use]
extern crate futures_ext;
extern crate mercurial;
extern crate mercurial_types;
#[macro_use]
extern crate slog;
extern crate slog_glog_fmt;
extern crate tokio_core;

mod changeset;

use std::fs;
use std::path::Path;
use std::sync::Arc;

use clap::{App, Arg, ArgMatches};
use failure_ext::err_msg;
use futures::{Future, Stream};
use slog::{Drain, Logger};
use slog_glog_fmt::default_drain as glog_drain;
use tokio_core::reactor::{Core, Remote};

use blobrepo::BlobRepo;
use changesets::SqliteChangesets;
use mercurial::RevlogRepo;
use mercurial_types::RepositoryId;

fn setup_app<'a, 'b>() -> App<'a, 'b> {
    App::new("revlog to blob importer")
        .version("0.0.0")
        .about("make blobs")
        .args_from_usage(
            r#"
            <INPUT>                    'input revlog repo'
            --repo_id <repo_id>        'ID of the newly imported repo'
            --manifold-bucket [BUCKET] 'manifold bucket'
            --db-address [address]     'address of a db. Used only for manifold blobstore'
            --blobstore-cache-size [SIZE] 'size of the blobstore cache'
            --changesets-cache-size [SIZE] 'size of the changesets cache'
            [OUTPUT]                   'Blobstore output'
        "#,
        )
        .arg(
            Arg::with_name("blobstore")
                .long("blobstore")
                .takes_value(true)
                .possible_values(&["rocksdb", "manifold"])
                .required(true)
                .help("blobstore type"),
        )
}

fn open_blobrepo<'a>(logger: &Logger, remote: Remote, matches: &ArgMatches<'a>) -> BlobRepo {
    let repo_id = RepositoryId::new(matches.value_of("repo_id").unwrap().parse().unwrap());

    match matches.value_of("blobstore").unwrap() {
        "rocksdb" => {
            let output = matches.value_of("OUTPUT").expect("output is not specified");
            let output = Path::new(output)
                .canonicalize()
                .expect("Failed to read output path");

            assert!(
                output.is_dir(),
                "The path {:?} does not exist or is not a directory",
                output
            );

            for subdir in &[".hg", "blobs", "books", "heads"] {
                let subdir = output.join(subdir);
                if subdir.exists() {
                    assert!(
                        subdir.is_dir(),
                        "Failed to start Rocksdb BlobRepo: \
                         {:?} already exists and is not a directory",
                        subdir
                    );
                    let content: Vec<_> = subdir
                        .read_dir()
                        .expect("Failed to read directory content")
                        .collect();
                    assert!(
                        content.is_empty(),
                        "Failed to start Rocksdb BlobRepo: \
                         {:?} already exists and is not empty: {:?}",
                        subdir,
                        content
                    );
                } else {
                    fs::create_dir(&subdir)
                        .expect(&format!("Failed to create directory {:?}", subdir));
                }
            }

            {
                let changesets_path = output.join("changesets");
                assert!(
                    !changesets_path.exists(),
                    "Failed to start Rocksdb BlobRepo: {:?} already exists"
                );
                SqliteChangesets::create(changesets_path.to_string_lossy())
                    .expect("Failed to initialize changests sqlite database");
            }

            BlobRepo::new_rocksdb(
                logger.new(o!["BlobRepo:Rocksdb" => output.to_string_lossy().into_owned()]),
                &output,
                repo_id,
            ).expect("failed to create rocksdb blobrepo")
        }
        "manifold" => {
            let manifold_bucket = matches
                .value_of("manifold-bucket")
                .expect("manifold bucket is not specified");

            BlobRepo::new_test_manifold(
                logger.new(o!["BlobRepo:TestManifold" => manifold_bucket.to_owned()]),
                manifold_bucket,
                "new_blobimport_test",
                &remote,
                repo_id,
                matches
                    .value_of("db-address")
                    .expect("--db-address is not specified"),
                matches
                    .value_of("blobstore-cache-size")
                    .map(|val| val.parse::<usize>().expect("cache size must be integer"))
                    .unwrap_or(100_000_000),
                matches
                    .value_of("changesets-cache-size")
                    .map(|val| val.parse::<usize>().expect("cache size must be integer"))
                    .unwrap_or(100_000_000),
            ).expect("failed to create manifold blobrepo")
        }
        bad => panic!("unexpected blobstore type: {}", bad),
    }
}

fn main() {
    let matches = setup_app().get_matches();

    let revlogrepo = {
        let input = matches.value_of("INPUT").expect("input is not specified");
        RevlogRepo::open(input).expect("cannot open revlogrepo")
    };

    let mut core = Core::new().expect("cannot create tokio core");

    let drain = glog_drain().fuse();
    let logger = Logger::root(drain, o![]);

    let blobrepo = Arc::new(open_blobrepo(&logger, core.remote(), &matches));

    let csstream = changeset::upload_changesets(revlogrepo, blobrepo);

    core.run(csstream.for_each(|cs| {
        cs.map(|cs| {
            info!(logger, "inserted: {}", cs.get_changeset_id());
            ()
        }).map_err(|err| {
            error!(logger, "failed to blobimport: {}", err);

            for cause in err.causes() {
                info!(logger, "cause: {}", cause);
            }
            info!(logger, "root cause: {:?}", err.root_cause());

            let msg = format!("failed to blobimport: {}", err);
            err_msg(msg)
        })
    })).expect("main stream failed");
}
