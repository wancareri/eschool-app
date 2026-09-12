#import <Foundation/Foundation.h>
#import <os/log.h>

void eschool_nslog(const char *msg) {
    if (msg == NULL) return;
    os_log_info(OS_LOG_DEFAULT, "%{public}s", msg);
}
