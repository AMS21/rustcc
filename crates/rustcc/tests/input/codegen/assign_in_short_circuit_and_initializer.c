// RUN: ${{rustcc}} --print-ir

int a(void) {
  int a = 0 && (a = 5);
  a;
}
