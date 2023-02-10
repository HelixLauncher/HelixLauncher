#include <stdint.h>

#ifdef _WIN32
#define DLLEXPORT __declspec(dllexport)
#pragma comment(linker, "/EXPORT:NvOptimusEnablement,DATA")
#pragma comment(linker, "/EXPORT:AmdPowerXpressRequestHighPerformance,DATA")
#else
#define DLLEXPORT __attribute__((visibility("default")))
#endif

DLLEXPORT uint32_t NvOptimusEnablement = 1;
DLLEXPORT int AmdPowerXpressRequestHighPerformance = 1;
