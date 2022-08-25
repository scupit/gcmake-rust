#include <catch2/catch_session.hpp>
#include <catch2/catch_test_macros.hpp>

// This is one of several main function examples from the catch2 docs page:
// https://github.com/catchorg/Catch2/blob/devel/docs/own-main.md
int main(int argc, char* argv[]) {
  Catch::Session session; // There must be exactly one instance

  // writing to session.configData() here sets defaults
  // this is the preferred way to set them

  int returnCode = session.applyCommandLine( argc, argv );
  if( returnCode != 0 ) // Indicates a command line error
        return returnCode;

  // writing to session.configData() or session.Config() here
  // overrides command line args
  // only do this if you know you need to

  int numFailed = session.run();

  // numFailed is clamped to 255 as some unices only use the lower 8 bits.
  // This clamping has already been applied, so just return it here
  // You can also do any post run clean-up here
  return numFailed;
}

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
