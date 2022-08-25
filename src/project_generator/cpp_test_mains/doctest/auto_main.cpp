#define DOCTEST_CONFIG_IMPLEMENT_WITH_MAIN
#include "doctest/doctest.h"

// If testing an executable, all code and dependencies are already included in this test by default.
// In any case, just include any headers from the project as usual.

// NOTE: When defining tests in other files, just #include "doctest/doctest.h" on its own.
// Don't add another DOCTEST_CONFIG_IMPLEMENT_WITH_MAIN define.

template<typename T>
T doubleIt(const T& item) {
  return item * 2;
}

TEST_CASE( "Can numbers be doubled" ) {
  CHECK(doubleIt(2.0) == 4.0);
  CHECK(doubleIt(3) == 6);
}
