// treemanifest.cpp - c++ implementation of a tree manifest
//
// Copyright 2016 Facebook, Inc.
//
// This software may be used and distributed according to the terms of the
// GNU General Public License version 2 or any later version.
#include <Python.h>
#include <ctype.h>
#include <iostream>
#include <string>
#include <vector>

typedef struct {
  PyObject_HEAD;

  // A reference to the store that is used to fetch new content
  PyObject *store;

  // The 20-byte root node of this manifest
  std::string node;
} treemanifest;

/*
 * Deallocates the contents of the treemanifest
 * */
static void treemanifest_dealloc(treemanifest *self){
  self->node.std::string::~string();
  Py_XDECREF(self->store);
  PyObject_Del(self);
}

/*
 * Initializes the contents of a treemanifest
 * */
static int treemanifest_init(treemanifest *self, PyObject *args) {
  PyObject *store;
  char *node;
  int nodelen;

  if (!PyArg_ParseTuple(args, "Os#", &store, &node, &nodelen)) {
    return -1;
  }

  self->store = store;
  Py_INCREF(store);
  // We have to manually call the member constructor, since the provided 'self'
  // is just zerod out memory.
  try {
    new (&self->node) std::string(node, nodelen);
  } catch (const std::exception &ex) {
    PyErr_SetString(PyExc_RuntimeError, ex.what());
    return -1;
  }

  return 0;
}

// ====  treemanifest ctype declaration ====

static PyMethodDef treemanifest_methods[] = {
  {NULL, NULL}
};

static PyTypeObject treemanifestType = {
  PyObject_HEAD_INIT(NULL)
  0,                                                /* ob_size */
  "ctreemanifest.treemanifest",                     /* tp_name */
  sizeof(treemanifest),                             /* tp_basicsize */
  0,                                                /* tp_itemsize */
  (destructor)treemanifest_dealloc,                 /* tp_dealloc */
  0,                                                /* tp_print */
  0,                                                /* tp_getattr */
  0,                                                /* tp_setattr */
  0,                                                /* tp_compare */
  0,                                                /* tp_repr */
  0,                                                /* tp_as_number */
  0,                                                /* tp_as_sequence - length/contains */
  0,                                                /* tp_as_mapping - getitem/setitem*/
  0,                                                /* tp_hash */
  0,                                                /* tp_call */
  0,                                                /* tp_str */
  0,                                                /* tp_getattro */
  0,                                                /* tp_setattro */
  0,                                                /* tp_as_buffer */
  Py_TPFLAGS_DEFAULT,                               /* tp_flags */
  "TODO",                                           /* tp_doc */
  0,                                                /* tp_traverse */
  0,                                                /* tp_clear */
  0,                                                /* tp_richcompare */
  0,                                                /* tp_weaklistoffset */
  0,                                                /* tp_iter */
  0,                                                /* tp_iternext */
  treemanifest_methods,                             /* tp_methods */
  0,                                                /* tp_members */
  0,                                                /* tp_getset */
  0,                                                /* tp_base */
  0,                                                /* tp_dict */
  0,                                                /* tp_descr_get */
  0,                                                /* tp_descr_set */
  0,                                                /* tp_dictoffset */
  (initproc)treemanifest_init,                      /* tp_init */
  0,                                                /* tp_alloc */
};

static PyMethodDef mod_methods[] = {
  {NULL, NULL}
};

static char mod_description[] = "Module containing a native treemanifest implementation";

PyMODINIT_FUNC initctreemanifest(void)
{
  PyObject *mod;

  treemanifestType.tp_new = PyType_GenericNew;
  if (PyType_Ready(&treemanifestType) < 0) {
    return;
  }

  mod = Py_InitModule3("ctreemanifest", mod_methods, mod_description);
  Py_INCREF(&treemanifestType);
  PyModule_AddObject(mod, "treemanifest", (PyObject *)&treemanifestType);
}
