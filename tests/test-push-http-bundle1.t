  $ . helpers-usechg.sh

#require killdaemons

This test checks behavior related to bundle1 that changed or is likely
to change with bundle2. Feel free to factor out any part of the test
which does not need to exist to keep bundle1 working.

  $ cat << EOF >> $HGRCPATH
  > [devel]
  > # This test is dedicated to interaction through old bundle
  > legacy.exchange = bundle1
  > EOF

  $ hg init test
  $ cd test
  $ echo a > a
  $ hg ci -Ama
  adding a
  $ cd ..
  $ hg clone test test2
  updating to branch default
  1 files updated, 0 files merged, 0 files removed, 0 files unresolved
  $ cd test2
  $ echo a >> a
  $ hg ci -mb
  $ req() {
  >     hg serve -p 0 --port-file .p -d --pid-file=hg.pid -E errors.log
  >     HGPORT=`cat .p`
  >     rm .p
  >     cat hg.pid >> $DAEMON_PIDS
  >     hg --cwd ../test2 push http://localhost:$HGPORT/
  >     exitstatus=$?
  >     killdaemons.py
  >     echo % serve errors
  >     cat errors.log
  >     return $exitstatus
  > }
  $ cd ../test

expect ssl error

  $ req
  pushing to http://localhost:$HGPORT/ (glob)
  searching for changes
  devel-warn: using deprecated bundlev1 format
   at: */changegroup.py:* (makechangegroup) (glob)
  abort: HTTP Error 403: ssl required
  % serve errors
  [255]

expect authorization error

  $ echo '[web]' > .hg/hgrc
  $ echo 'push_ssl = false' >> .hg/hgrc
  $ req
  pushing to http://localhost:$HGPORT/ (glob)
  searching for changes
  devel-warn: using deprecated bundlev1 format
   at: */changegroup.py:* (makechangegroup) (glob)
  abort: authorization failed
  % serve errors
  [255]

expect authorization error: must have authorized user

  $ echo 'allow_push = unperson' >> .hg/hgrc
  $ req
  pushing to http://localhost:$HGPORT/ (glob)
  searching for changes
  devel-warn: using deprecated bundlev1 format
   at: */changegroup.py:* (makechangegroup) (glob)
  abort: authorization failed
  % serve errors
  [255]

expect success

  $ cat >> .hg/hgrc <<EOF
  > allow_push = *
  > [hooks]
  > changegroup = sh -c "printenv.py changegroup 0"
  > pushkey = sh -c "printenv.py pushkey 0"
  > EOF
  $ req
  pushing to http://localhost:$HGPORT/ (glob)
  searching for changes
  devel-warn: using deprecated bundlev1 format
   at: */changegroup.py:* (makechangegroup) (glob)
  remote: adding changesets
  remote: adding manifests
  remote: adding file changes
  remote: added 1 changesets with 1 changes to 1 files
  remote: changegroup hook: HG_HOOKNAME=changegroup HG_HOOKTYPE=changegroup HG_NODE=ba677d0156c1196c1a699fa53f390dcfc3ce3872 HG_NODE_LAST=ba677d0156c1196c1a699fa53f390dcfc3ce3872 HG_SOURCE=serve HG_TXNID=TXN:$ID$ HG_URL=remote:http:$LOCALIP: (glob)
  % serve errors
  $ hg rollback
  repository tip rolled back to revision 0 (undo serve)

expect success, server lacks the httpheader capability

  $ CAP=httpheader
  $ . "$TESTDIR/notcapable"
  $ req
  pushing to http://localhost:$HGPORT/ (glob)
  searching for changes
  devel-warn: using deprecated bundlev1 format
   at: */changegroup.py:* (makechangegroup) (glob)
  remote: adding changesets
  remote: adding manifests
  remote: adding file changes
  remote: added 1 changesets with 1 changes to 1 files
  remote: changegroup hook: HG_HOOKNAME=changegroup HG_HOOKTYPE=changegroup HG_NODE=ba677d0156c1196c1a699fa53f390dcfc3ce3872 HG_NODE_LAST=ba677d0156c1196c1a699fa53f390dcfc3ce3872 HG_SOURCE=serve HG_TXNID=TXN:$ID$ HG_URL=remote:http:$LOCALIP: (glob)
  % serve errors
  $ hg rollback
  repository tip rolled back to revision 0 (undo serve)

expect success, server lacks the unbundlehash capability

  $ CAP=unbundlehash
  $ . "$TESTDIR/notcapable"
  $ req
  pushing to http://localhost:$HGPORT/ (glob)
  searching for changes
  devel-warn: using deprecated bundlev1 format
   at: */changegroup.py:* (makechangegroup) (glob)
  remote: adding changesets
  remote: adding manifests
  remote: adding file changes
  remote: added 1 changesets with 1 changes to 1 files
  remote: changegroup hook: HG_HOOKNAME=changegroup HG_HOOKTYPE=changegroup HG_NODE=ba677d0156c1196c1a699fa53f390dcfc3ce3872 HG_NODE_LAST=ba677d0156c1196c1a699fa53f390dcfc3ce3872 HG_SOURCE=serve HG_TXNID=TXN:$ID$ HG_URL=remote:http:$LOCALIP: (glob)
  % serve errors
  $ hg rollback
  repository tip rolled back to revision 0 (undo serve)

expect success, pre-d1b16a746db6 server supports the unbundle capability, but
has no parameter

  $ cat <<EOF > notcapable-unbundleparam.py
  > from mercurial import extensions, httppeer
  > def capable(orig, self, name):
  >     if name == 'unbundle':
  >         return True
  >     return orig(self, name)
  > def uisetup(ui):
  >     extensions.wrapfunction(httppeer.httppeer, 'capable', capable)
  > EOF
  $ cp $HGRCPATH $HGRCPATH.orig
  $ cat <<EOF >> $HGRCPATH
  > [extensions]
  > notcapable-unbundleparam = `pwd`/notcapable-unbundleparam.py
  > EOF
  $ req
  pushing to http://localhost:$HGPORT/ (glob)
  searching for changes
  devel-warn: using deprecated bundlev1 format
   at: */changegroup.py:* (makechangegroup) (glob)
  remote: adding changesets
  remote: adding manifests
  remote: adding file changes
  remote: added 1 changesets with 1 changes to 1 files
  remote: changegroup hook: * (glob)
  % serve errors
  $ hg rollback
  repository tip rolled back to revision 0 (undo serve)
  $ mv $HGRCPATH.orig $HGRCPATH

expect push success, phase change failure

  $ cat > .hg/hgrc <<EOF
  > [web]
  > push_ssl = false
  > allow_push = *
  > [hooks]
  > prepushkey = sh -c "printenv.py prepushkey 1"
  > EOF
  $ req
  pushing to http://localhost:$HGPORT/ (glob)
  searching for changes
  devel-warn: using deprecated bundlev1 format
   at: */changegroup.py:* (makechangegroup) (glob)
  remote: adding changesets
  remote: adding manifests
  remote: adding file changes
  remote: added 1 changesets with 1 changes to 1 files
  % serve errors

expect phase change success

  $ cat >> .hg/hgrc <<EOF
  > prepushkey = sh -c "printenv.py prepushkey 0"
  > EOF
  $ req
  pushing to http://localhost:$HGPORT/ (glob)
  searching for changes
  no changes found
  % serve errors
  [1]
  $ hg rollback
  repository tip rolled back to revision 0 (undo serve)

expect authorization error: all users denied

  $ echo '[web]' > .hg/hgrc
  $ echo 'push_ssl = false' >> .hg/hgrc
  $ echo 'deny_push = *' >> .hg/hgrc
  $ req
  pushing to http://localhost:$HGPORT/ (glob)
  searching for changes
  devel-warn: using deprecated bundlev1 format
   at: */changegroup.py:* (makechangegroup) (glob)
  abort: authorization failed
  % serve errors
  [255]

expect authorization error: some users denied, users must be authenticated

  $ echo 'deny_push = unperson' >> .hg/hgrc
  $ req
  pushing to http://localhost:$HGPORT/ (glob)
  searching for changes
  devel-warn: using deprecated bundlev1 format
   at: */changegroup.py:* (makechangegroup) (glob)
  abort: authorization failed
  % serve errors
  [255]

  $ cd ..
