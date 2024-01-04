(function () {
  // Modules will be added here
  const modules = {
    "/Users/darrenkim/Desktop/study/rust/toy_js_bundler/test/index.js":
      function (exports, require) {
        const {
          default: foo,
        } = require("/Users/darrenkim/Desktop/study/rust/toy_js_bundler/test/foo.js");
        const {
          default: boo,
        } = require("/Users/darrenkim/Desktop/study/rust/toy_js_bundler/test/boo.js");

        const result = foo(10);
        const booResult = boo(5);

        console.log(result);
        console.log(booResult);
      },
    "/Users/darrenkim/Desktop/study/rust/toy_js_bundler/test/foo.js": function (
      exports,
      require
    ) {
      const {
        default: boo,
      } = require("/Users/darrenkim/Desktop/study/rust/toy_js_bundler/test/boo.js");

      function foo(x) {
        return boo(x);
      }

      exports.default = foo;
    },
    "/Users/darrenkim/Desktop/study/rust/toy_js_bundler/test/boo.js": function (
      exports,
      require
    ) {
      function boo(test) {
        return test * test;
      }

      exports.default = boo;
    },
    "/Users/darrenkim/Desktop/study/rust/toy_js_bundler/test/boo.js": function (
      exports,
      require
    ) {
      function boo(test) {
        return test * test;
      }

      exports.default = boo;
    },
  };
  const entry =
    "/Users/darrenkim/Desktop/study/rust/toy_js_bundler/test/index.js";

  // Module cache to store instantiated modules
  const moduleCache = {};

  // Custom require function to load modules
  const require = (moduleName) => {
    // Check if module is in cache
    if (moduleCache[moduleName]) {
      return moduleCache[moduleName].exports;
    }

    // If not, initialize and load the module
    const module = { exports: {} };
    moduleCache[moduleName] = module;

    try {
      modules[moduleName](module.exports, require);
    } catch (error) {
      throw new Error(
        "Module load error in " + moduleName + ": " + error.message
      );
    }

    // Return the exports from the module
    return module.exports;
  };

  // Start the application by requiring the entry module
  try {
    require(entry);
  } catch (error) {
    console.error("Application failed to start: " + error.message);
  }
})();
