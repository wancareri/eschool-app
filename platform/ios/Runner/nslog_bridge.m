// NSLog bridge — routes Rust log output through NSLog with %{public}s
// so idevicesyslog can see the messages (iOS redacts without %{public}s).

#import <Foundation/Foundation.h>

void eschool_nslog(const char *msg) {
    if (msg == NULL) return;
    NSLog(@"%{public}s", msg);
}
