public class MakeJVM {
  static int compute(int n) {
    int sum = 0;
    for (int i = 1; i <= n; i++) {
      sum += i;
    }
    return sum;
  }

  static int doubled(int n) {
    return n * 2;
  }

  static int compute2(int n) {
    int sum = 0;
    for (int i = 1; i <= n; i++) {
      sum += doubled(i);
    }
    return sum;
  }

  static boolean isEven(int n) {
    if (n == 0) {
      return true;
    }
    if (n == 1) {
      return false;
    }
    return isOdd(n - 1);
  }

  static boolean isOdd(int n) {
    if (n == 0) {
      return false;
    }
    if (n == 1) {
      return true;
    }
    return isEven(n - 1);
  }

  static int start() {
    return compute(10);
  }

  static int start2() {
    return compute2(10);
  }

  static boolean start3() {
    return isEven(50);
  }
}