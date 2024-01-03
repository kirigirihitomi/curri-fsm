typedef void *(*fn)(void *);
typedef struct Closure
{
    void *a;
    void *b;
} Closure; // rust fat pointer

extern "C" void *CurriMachine(void *, const char *);
extern "C" void *CurriDropMachine(void *);
extern "C" Closure CurriState(const char *, fn, fn);
extern "C" Closure CurriTransitions(const char *, const char *, const char *);
extern "C" Closure CurriTrigger(const char *);
extern "C" Closure CurriCompose(Closure *, int);
extern "C" void *CurriRun(Closure, void *);