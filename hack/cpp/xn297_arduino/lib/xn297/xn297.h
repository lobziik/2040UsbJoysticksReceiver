#include <stdio.h>

// values to be OR-ed with the register address for read, write:
// these are same for all transceiver modules
#define regReadCmd 0x00
#define regWriteCmd 0x20

#define XN297_DEBUG_OUT 1

typedef enum
{
    data_rate_2M = 0,
    data_rate_1M,
    data_rate_250k
} dataRate;


class Xn297
{

private:
    uint8_t irq, miso, mosi, sck, csn, ce; // pins for software SPI
    uint8_t addressWidth;
    void usePins(int pIRQ, int pMISO, int pMOSI, int pSCK, int pCSN, int pCE);
    void spiInit();

    uint8_t spiTransfer(uint8_t data);

    static uint8_t bitReverse(uint8_t b_in);
    static void bitReverseEachElement(uint8_t *buf, uint8_t bufLen);

    void (*onReceive)(void);
	void (*onDebug)(const char* s);

    void debug(const char* s);
	static void dummyOnDebug(const char* s);

public:
    Xn297();
    void begin(uint8_t irq, uint8_t miso, uint8_t mosi, uint8_t sck, uint8_t csn, uint8_t ce);
    void readSettings();

    void ceHigh();
    void ceLow();

    void readRegister(uint8_t reg, uint8_t *dest, uint8_t len);
    uint8_t readRegister(uint8_t reg);

    void writeCommand(uint8_t cmd, uint8_t *data, uint8_t len);
    void writeCommand(uint8_t cmd, uint8_t data);

    void writeRegister(uint8_t reg, uint8_t *data, uint8_t len);
    void writeRegister(uint8_t reg, uint8_t data);

    void setBit(uint8_t reg, uint8_t b);
    void clearBit(uint8_t reg, uint8_t b);
    uint8_t readBit(uint8_t reg, uint8_t b);

    void setRxPayloadWidth(uint8_t pipeNumber, uint8_t payloadW);
    uint8_t readRxPayloadWidth();
	void readRxPayload(uint8_t *dest, uint8_t len);

    void setCh(uint8_t ch);
	void setModeRX();
    void powerUp();
	void shutDown();
	void setCRCSize(uint8_t crcSize);

    void enableDynAck();
	void disableDynAck();

    void setDataRate(dataRate);
	dataRate getDataRate();
		
	void tick();
	void setOnReceive( void (*f)(void));

    void setOnDebug( void (*f)(const char* s));
};
