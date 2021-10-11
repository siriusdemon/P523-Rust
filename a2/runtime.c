#include <stdlib.h>
#include <stdio.h>

#ifdef APPLE /* MacOS */
#define SCHEME_ENTRY scheme_entry
#else
#define SCHEME_ENTRY _scheme_entry
#endif

extern long SCHEME_ENTRY(void);

void print(long x){
  printf("%ld\n", x);
}

int main(int argc,char **argv){
  /*no arguments at this point */
  if (argc != 1) {
    fprintf(stderr, "usage: %s\n",argv[0]);
    exit(1);
  }

  print(SCHEME_ENTRY());
  return 0;  
}
