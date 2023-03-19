
#include "inline.h"

#include "Recast.h"

rcContext* CreateContext(bool state) { return new rcContext(state); }

void DeleteContext(rcContext* context) { delete context; }
