#ifdef PKG_CONFIG

/* Just use installed headers */
#include <bzlib.h>

#else // #ifdef PKG_CONFIG

#include "bzip2-1.0.8/bzlib.h"

#endif // #ifdef PKG_CONFIG


/* This file is used to generate bindings for both headers.
 * Check update_bindings.sh to see how to use it.
 * Or use the `bindgen` feature, which will create the bindings automatically. */
