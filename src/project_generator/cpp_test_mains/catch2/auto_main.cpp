#include <catch2/catch_test_macros.hpp>

template<typename T>
T doubleIt(const T& item) {
  return item * 2;
}

TEST_CASE( "Can double things", "[doubling]" ) {
  SECTION( "Doubling" ) {
    REQUIRE( doubleIt(2) == 4 );
    REQUIRE( doubleIt(3.0) == 3.0 * 2 );
    REQUIRE( doubleIt(5.0) == 5.0 * 2 );
  }
}
