// RUN: ${{rustcc}} --print-ast
// EXPECT-FAILURE

int f1(void) { int b; }

int f2(void) { b; }
