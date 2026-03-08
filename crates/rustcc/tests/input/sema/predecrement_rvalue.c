// RUN: ${{rustcc}} --print-ast
// EXPECT-FAILURE

int p(void) { --1; }