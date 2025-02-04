/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

import '@testing-library/jest-dom';

// Use __mocks__/logger so calls to logger don't output to console, but
// console.log still works for debugging tests.
jest.mock('./logger');

import {configure} from '@testing-library/react';

if (process.env.HIDE_RTL_DOM_ERRORS) {
  configure({
    getElementError: (message: string | null) => {
      const error = new Error(message ?? '');
      error.name = 'TestingLibraryElementError';
      error.stack = null;
      return error;
    },
  });
}
