class TestClass {
  /**
   * This is a JavaDoc with <i>many</i> of the fancy frills that come with it.
   *
   * <p>
   * Notably, the allowed use of HTML inline to <i>format</i> the text.
   * </p>
   *
   * Also, the allowed use of the various metadata tags we can attach to methods
   * and classes.
   *
   * @param args these are the arguents passed to the program from the command
   *             lin.
   */
  public static void main(String[] args) {
    greet("world");
  }

  /**
   * This doc has a link in it: {@link this sould b ignor} but not tis
   *
   * @param name this is an other test.
   */
  public static void greet(String name) {
    System.out.println("Hello " + name + ".");
  }
}
