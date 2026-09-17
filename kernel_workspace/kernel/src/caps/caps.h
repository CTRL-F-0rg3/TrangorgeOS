#ifndef TRANGORGE_CAPS_H
#define TRANGORGE_CAPS_H


#define SYS_CAP_QUERY    0x1070ull 
#define SYS_CAP_REQUEST  0x1071ull 
#define SYS_CAP_RELEASE  0x1072ull 
#define SYS_CAP_AUDIT    0x1073ull 

unsigned int caps_self_bits(void);

unsigned int caps_world_bits(unsigned int world_id);

int caps_has(unsigned int world_id, unsigned char cap_id);

int caps_name(unsigned char cap_id, unsigned char *buf, unsigned int len);

int caps_request(unsigned int target, unsigned char cap_id);

int caps_release(unsigned int world_id, unsigned char cap_id);

unsigned int caps_world_count(void);

unsigned long long caps_audit_count(void);

#endif 