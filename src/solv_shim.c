#include <solv/pool.h>
#include <solv/repo.h>
#include <solv/solvable.h>

int cnf_pool_installable(Pool *pool, Solvable *s) {
    return pool_installable(pool, s);
}
