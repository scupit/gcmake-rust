#include "gtest/gtest.h"

template<typename T>
T doubleIt(const T& item) {
  return item * 2;
}

TEST(HelloTest, CanDoubleThings) {
  EXPECT_EQ(doubleIt(2), 4);
  EXPECT_EQ(doubleIt(3.0), 3.0 * 2);
}
