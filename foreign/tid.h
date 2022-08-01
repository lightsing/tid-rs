#ifndef TID_RS_TID_H
#define TID_RS_TID_H

#include "stdint.h"

void* create_la_context();
void drop_la_context(void* ctx);
void set_localized_cancel_title(void* ctx, char* reason);
int32_t can_evaluate_policy(void *ctx, int32_t policy);
void evaluate_policy(void* ctx, int32_t policy, char* reason, void* user_data, void* callback);

#endif
