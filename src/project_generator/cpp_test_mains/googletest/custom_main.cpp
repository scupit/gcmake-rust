#include "gtest/gtest.h"

// Example main is found in the googletest primer page:
// https://google.github.io/googletest/primer.html#writing-the-main-function

int main(int argc, char** argv) {
  testing::InitGoogleTest(&argc, argv);
  return RUN_ALL_TESTS();
}

template<typename T>
T doubleIt(const T& item) {
  return item * 2;
}

TEST(HelloTest, CanDoubleThings) {
  EXPECT_EQ(doubleIt(2), 4);
  EXPECT_EQ(doubleIt(3.0), 3.0 * 2);
}
