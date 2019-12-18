#ifndef _PLATFORM_H_
#define _PLATFORM_H_

#if defined(CONFIG_H_FILE)
#include CONFIG_H_FILE
#elif defined(HAVE_CONFIG_H)
#include "config.h"
#else
#error Need either CONFIG_H_FILE or HAVE_CONFIG_H defined.
#endif

#endif /* !_PLATFORM_H_ */
