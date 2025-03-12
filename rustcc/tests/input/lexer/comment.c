// RUN: ${{rustcc}} --print-tokens

// This is a comment
 // Hey, this as well

int main(void) { // Comments here
    return 0; // Comments there
} // Comments everywhere
