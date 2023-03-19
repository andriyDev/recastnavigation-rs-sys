#pragma once

class rcContext;

rcContext* CreateContext(bool state = true);

void DeleteContext(rcContext* context);
