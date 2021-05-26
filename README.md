# Desktop IP to Arduino Morse code

This is a combination of a desktop companion and Arduino software.
- The desktop companion periodically polls for connected Arduino modules running the software in this repo.
- After it is found, it will send all the IPs matched by arguments in the desktop companion or found interfaces otherwise. 
- Once the Arduino receives the IP, it will blink in morse code. 
- After it is done, it will keep the LED on for 4 seconds to indicate it's done. 

This communicates using the USB serial port and sends a `fernocat\n` ping every 500 ms to the host. The Arduino will relay back the message twice:
- First to start
- Second when finished blinking and waiting 4 seconds

## Build
### Desktop Companion
Simply just run `cargo build`. On Windows, you might need to follow the [steps for libpnet](https://github.com/libpnet/libpnet#windows) to build.

### Arduino software
While this is using a PlatformIO template, you can theoretically just copy the `main.cpp` into Arduino IDE, and it _should_ work.