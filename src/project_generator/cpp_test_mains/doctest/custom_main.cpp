// This is a slightly modified version of the example main entrypoint from the docs:
// https://github.com/doctest/doctest/blob/master/doc/markdown/main.md

#define DOCTEST_CONFIG_IMPLEMENT
#include "doctest/doctest.h"

int main(int argc, char** argv) {
  doctest::Context context;

  // Filter and option reference is here:
  // https://github.com/doctest/doctest/blob/master/doc/markdown/commandline.md

  // Example defaults
  context.addFilter("test-case-exclude", "*math*"); // exclude test cases with "math" in their name
  context.setOption("abort-after", 5); // stop test execution after 5 failed assertions
  context.setOption("order-by", "name"); // sort the test cases by their name

  context.applyCommandLine(argc, argv);

  // Example overrides
  context.setOption("no-breaks", true); // don't break in the debugger when assertions fail

  // Run the test using the configured context
  int testResultCode = context.run();

  if(context.shouldExit()) // important - query flags (and --exit) rely on the user doing this
    return testResultCode; // propagate the result of the tests
  
  int programReturnCode = 0;
  // Here, the "rest of the program" can be run. 
  
  return testResultCode + programReturnCode; // the result from doctest is propagated here as well
}

// If testing an executable, all code and dependencies are already included in this test by default.
// In any case, just include any headers from the project as usual.

// NOTE: When defining tests in other files, just #include "doctest/doctest.h" on its own.
// Don't add another DOCTEST_CONFIG_IMPLEMENT define.

template<typename T>
T doubleIt(const T& item) {
  return item * 2;
}

TEST_CASE( "Can numbers be doubled" ) {
  CHECK(doubleIt(2.0) == 4.0);
  CHECK(doubleIt(3) == 6);
}
