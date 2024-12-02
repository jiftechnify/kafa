public class StaticFieldsSample {
  // compiled as constant in const pool, loaded by ldc
  public static final int ANSWER = 424242;

  // to be initialized to default value (0)
  static int fooCount;
  // to be initialized via class initialization
  static int barCount = 1;

  static void foo() {
    fooCount++;
    barCount <<= 1;
  }

  static int start() {
    for (int i = 0; i < 10; i++) {
      foo();
    }
    // 10 + 1024 + 424242 = 425276
    return fooCount + barCount + ANSWER; 
  }
}
