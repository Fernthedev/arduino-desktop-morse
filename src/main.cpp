#include <Arduino.h>

#define WORD_PAUSE delay(1300);
#define CHAR_PAUSE delay(700);
#define SHORT_PAUSE delay(250);
#define LONG_PAUSE delay(600);
#define LONG_PAUSE_END delay(450);

#define DO_DEBUG

#ifdef DO_DEBUG
#define debugLog(x)         Serial.println(x);
#elif
#define debugLog(x) ((void)0)
#endif

int RelayPin = 4;

int serialBitrate = 9600;

void setup()
{

    // initialize LED digital pin as an output.
    pinMode(LED_BUILTIN, OUTPUT);

    // initialize pin for USB handshake
    pinMode((uint8_t) RelayPin, OUTPUT);

    Serial.begin(serialBitrate);
}
static String getSerialData() {
    String data = "";

    // Keep reading until end of line
    bool inputData = Serial.available() > 0;
    while (inputData) {
        while (Serial.available() > 0) {
            digitalWrite(LED_BUILTIN, HIGH);
            auto charData = char(Serial.read());

            if (charData == '\n') {
                digitalWrite(LED_BUILTIN, LOW);
                Serial.flush();

                return data;
            }

            data += charData;
        }
    }


    return data;
}

static String charToMorse(const char& input) {
    auto upperInput = (char) toUpperCase(input);


    static char text[36] = {'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
                     '1','2','3','4','5','6','7','8', '9','0' };

    static String morse[36] = {".-","-...","-.-.","-..",".","..-","--.","....","..",".---","-.-",".-..","--","-.","---",".--.","--.-",".-.","...","-","..-","...-",".--","-..-","-.--","--..",
                             ".----","..---","...--","....-",".....","-....","--....","---..","----.","-----"};

    // if only we could hash map this efficiently, oh well
    for (int i = 0; i < 36; i++) {
        if (text[i] == upperInput)
            return morse[i];
    }

    return "";
}

void morseToLight(String morse) {
    if (morse == "") return;

    digitalWrite(LED_BUILTIN, LOW);

    for (const auto& stringChar : morse) {
        if
        (stringChar == '.') {
            digitalWrite(LED_BUILTIN, HIGH);
            SHORT_PAUSE
            digitalWrite(LED_BUILTIN, LOW);
            SHORT_PAUSE
        }
        else if
        (stringChar == '-') {
            digitalWrite(LED_BUILTIN, HIGH);
            LONG_PAUSE
            digitalWrite(LED_BUILTIN, LOW);
            LONG_PAUSE_END
        } else if (stringChar == '/') {
            digitalWrite(LED_BUILTIN, HIGH);
            WORD_PAUSE
            digitalWrite(LED_BUILTIN, LOW);
            SHORT_PAUSE
        }


    }
}

void loop()
{
    String data = getSerialData();

    if (data != "") {
        Serial.println(data);
        delay(1000); // To indicate about to start morse code


        bool pause = false;

        for (const char &stringChar : data) {
            if (stringChar == ' ') {
                morseToLight("/");
                continue;
            }
            else if (pause)
                CHAR_PAUSE

            morseToLight(charToMorse(stringChar));
            pause = true;
        }

        SHORT_PAUSE
        digitalWrite(LED_BUILTIN, LOW);

        debugLog("Finished writing")

        delay(4000);

        // Reconfirm
        Serial.println(data);
    }
}