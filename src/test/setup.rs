use crate::environment::Environments;
use crate::test::eval::test_eval;

pub async fn setup_env(envs: &mut Environments) {
  let miscallenious = "
  var weblab_undefined = undefined;
  var weblab_null = null;
  var weblab_String = String;
  ";
  let stajs = "// Copyright (c) 2012 Ecma International.  All rights reserved.
      // This code is governed by the BSD license found in the LICENSE file.
      /*---
      description: |
          Provides both:
      
          - An error class to avoid false positives when testing for thrown exceptions
          - A function to explicitly throw an exception using the Test262Error class
      defines: [Test262Error, $ERROR, $DONOTEVALUATE]
      ---*/
      
      
      class Test262Error{
          constructor(message) {
              this.message = message || '';
          }
      }
      
      Test262Error.prototype.toString = function () {
        return 'Test262Error: ' + this.message;
      };
      
      var $ERROR = (...args) => {
        throw new Test262Error(...args);
      };
      
      function $DONOTEVALUATE() {
        throw 'Test262: This statement should not be evaluated.';
      }
      ";
  let weblab_stajs = "// Copyright (c) 2012 Ecma International.  All rights reserved.
      // This code is governed by the BSD license found in the LICENSE file.
      /*---
      description: |
          Provides both:
      
          - An error class to avoid false positives when testing for thrown exceptions
          - A function to explicitly throw an exception using the Test262Error class
      defines: [Test262Error, $ERROR, $DONOTEVALUATE]
      ---*/
      
      
      class weblab_Test262Error{
          constructor(message) {
              this.message = message || '';
          }
      }
      
      weblab_Test262Error.prototype.toString = function () {
        return 'Test262Error: ' + this.message;
      };
      
      var weblab_$ERROR = (...args) => {
        throw new weblab_Test262Error(...args);
      };
      
      function weblab_$DONOTEVALUATE() {
        throw 'Test262: This statement should not be evaluated.';
      }
      ";
  let assertjs = "// Copyright (C) 2017 Ecma International.  All rights reserved.
      // This code is governed by the BSD license found in the LICENSE file.
      /*---
      description: |
          Collection of assertion functions used throughout test262
      defines: [assert]
      ---*/
      
      
      class assert{
          constructor(mustBeTrue, message) {
        if (mustBeTrue === true) {
          return;
        }
      
        if (message === undefined) {
          message = 'Expected true but got ' + assert._toString(mustBeTrue);
        }
        $ERROR(message);
      }}
      
      assert._isSameValue = function (a, b) {
        if (a === b) {
          // Handle +/-0 vs. -/+0
          return a !== 0 || 1 / a === 1 / b;
        }
      
        // Handle NaN vs. NaN
        return a !== a && b !== b;
      };
      
      assert.sameValue = function (actual, expected, message) {
        try {
          if (assert._isSameValue(actual, expected)) {
            return;
          }
        } catch (error) {
          $ERROR(message + ' (_isSameValue operation threw) ' + error);
          return;
        }
      
        if (message === undefined) {
          message = '';
        } else {
          message += ' ';
        }
      
        message += 'Expected SameValue(«' + assert._toString(actual) + '», «' + assert._toString(expected) + '») to be true';
      
        $ERROR(message);
      };
      
      assert.notSameValue = function (actual, unexpected, message) {
        if (!assert._isSameValue(actual, unexpected)) {
          return;
        }
      
        if (message === undefined) {
          message = '';
        } else {
          message += ' ';
        }
      
        message += 'Expected SameValue(«' + assert._toString(actual) + '», «' + assert._toString(unexpected) + '») to be false';
      
        $ERROR(message);
      };
      
      assert.throws = function (expectedErrorConstructor, func, message) {
        if (typeof func !== \"function\") {
          $ERROR('assert.throws requires two arguments: the error constructor ' +
            'and a function to run');
          return;
        }
        if (message === undefined) {
          message = '';
        } else {
          message += ' ';
        }
      
        try {
          func();
        } catch (thrown) {
          if (typeof thrown !== 'object' || thrown === null) {
            message += 'Thrown value was not an object!';
            $ERROR(message);
          } else if (thrown.constructor !== expectedErrorConstructor) {
            message += 'Expected a ' + expectedErrorConstructor.name + ' but got a ' + thrown.constructor.name;
            $ERROR(message);
          }
          return;
        }
      
        message += 'Expected a ' + expectedErrorConstructor.name + ' to be thrown but no exception was thrown at all';
        $ERROR(message);
      };
      
      assert._toString = function (value) {
        try {
          if (value === 0 && 1 / value === -Infinity) {
            return '-0';
          }
      
          return String(value);
        } catch (err) {
          if (err.name === 'TypeError') {
            return Object.prototype.toString.call(value);
          }
      
          throw err;
        }
      };
      ";
  let weblab_assertjs = "// Copyright (C) 2017 Ecma International.  All rights reserved.
      // This code is governed by the BSD license found in the LICENSE file.
      /*---
      description: |
          Collection of assertion functions used throughout test262
      defines: [assert]
      ---*/
      
      
      class weblab_assert{
          constructor(mustBeTrue, message) {
        if (mustBeTrue === true) {
          return;
        }
      
        if (message === undefined) {
          message = 'Expected true but got ' + weblab_assert._toString(mustBeTrue);
        }
        weblab_$ERROR(message);
      }}
      
      weblab_assert._isSameValue = function (a, b) {
        if (a === b) {
          // Handle +/-0 vs. -/+0
          return a !== 0 || 1 / a === 1 / b;
        }
      
        // Handle NaN vs. NaN
        return a !== a && b !== b;
      };
      
      weblab_assert.sameValue = function (actual, expected, message) {
        try {
          if (weblab_assert._isSameValue(actual, expected)) {
            return;
          }
        } catch (error) {
          weblab_$ERROR(message + ' (_isSameValue operation threw) ' + error);
          return;
        }
      
        if (message === undefined) {
          message = '';
        } else {
          message += ' ';
        }
      
        message += 'Expected SameValue(«' + weblab_assert._toString(actual) + '», «' + weblab_assert._toString(expected) + '») to be true';
      
        weblab_$ERROR(message);
      };
      
      weblab_assert.notSameValue = function (actual, unexpected, message) {
        if (!weblab_assert._isSameValue(actual, unexpected)) {
          return;
        }
      
        if (message === undefined) {
          message = '';
        } else {
          message += ' ';
        }
      
        message += 'Expected SameValue(«' + weblab_assert._toString(actual) + '», «' + weblab_assert._toString(unexpected) + '») to be false';
      
        weblab_$ERROR(message);
      };
      
      weblab_assert.throws = function (expectedErrorConstructor, func, message) {
        if (typeof func !== \"function\") {
          weblab_$ERROR('weblab_assert.throws requires two arguments: the error constructor ' +
            'and a function to run');
          return;
        }
        if (message === undefined) {
          message = '';
        } else {
          message += ' ';
        }
      
        try {
          func();
        } catch (thrown) {
          if (typeof thrown !== 'object' || thrown === null) {
            message += 'Thrown value was not an object!';
            weblab_$ERROR(message);
          } else if (thrown.constructor !== expectedErrorConstructor) {
            message += 'Expected a ' + expectedErrorConstructor.name + ' but got a ' + thrown.constructor.name;
            weblab_$ERROR(message);
          }
          return;
        }
      
        message += 'Expected a ' + expectedErrorConstructor.name + ' to be thrown but no exception was thrown at all';
        weblab_$ERROR(message);
      };
      
      weblab_assert._toString = function (value) {
        try {
          if (value === 0 && 1 / value === -Infinity) {
            return '-0';
          }
      
          return String(value);
        } catch (err) {
          if (err.name === 'TypeError') {
            return Object.prototype.toString.call(value);
          }
      
          throw err;
        }
      };
      ";
  test_eval(miscallenious, envs).await.unwrap();
  test_eval(weblab_stajs, envs).await.unwrap();
  test_eval(weblab_assertjs, envs).await.unwrap();
  test_eval(stajs, envs).await.unwrap();
  test_eval(assertjs, envs).await.unwrap();
}
