// RUN: ${{rustcc}} --print-ir

int a(void) {
  int a = 1 || (a = 5);
  a;
}
