typedef void *(*fn)(void *);
typedef struct Closure
{
    void *a;
    void *b;
} Closure; // rust的闭包大小为 2 * usize
extern void *CurriMachine(void *, const char *);
extern void *CurriDropMachine(void *);
extern Closure CurriState(const char *, fn, fn);
extern Closure CurriTransitions(const char *, const char *, const char *);
extern Closure CurriTrigger(const char *);
extern Closure CurriCompose(Closure *, int);
extern void *CurriRun(Closure, void *);