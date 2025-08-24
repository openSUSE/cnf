#pragma once

#include <solv/pool.h>
#include <solv/repo.h>
#include <solv/repo_solv.h>
#include <solv/dataiterator.h>
#include <solv/knownid.h>

int cnf_pool_installable(Pool *pool, Solvable *s);
