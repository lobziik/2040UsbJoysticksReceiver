#include "xn297.h"
#include "Arduino.h"

#ifdef XN297_DEBUG_OUT
#include "printHelpers.h"
#endif

Xn297::Xn297()
{
    setOnDebug(dummyOnDebug);
}

void Xn297::usePins(int pIRQ, int pMISO, int pMOSI, int pSCK, int pCSN, int pCE)
{
    irq = pIRQ;
    miso = pMISO;
    mosi = pMOSI;
    sck = pSCK;
    csn = pCSN;
    ce = pCE;
}

void Xn297::spiInit()
{
    // assuming usePins(...) was called before

    pinMode(ce, OUTPUT);
    digitalWrite(ce, LOW);

    pinMode(csn, OUTPUT);
    digitalWrite(csn, HIGH);

    pinMode(sck, OUTPUT);
    digitalWrite(sck, LOW);

    pinMode(miso, INPUT);

    pinMode(mosi, OUTPUT);
    digitalWrite(mosi, LOW);

    pinMode(irq, INPUT);
}

uint8_t Xn297::spiTransfer(uint8_t data)
{ // software SPI
    uint8_t resData = 0;

    data = bitReverse(data);

    for (int i = 0; i < 8; i++)
    {
        if (data % 2 == 1)
        {
            digitalWrite(mosi, HIGH);
        }
        else
        {
            digitalWrite(mosi, LOW);
        }

        data = data >> 1;

        delayMicroseconds(1);

        digitalWrite(sck, HIGH);

        resData = resData << 1;
        resData = resData + digitalRead(miso);

        delayMicroseconds(1);

        digitalWrite(sck, LOW);

        delayMicroseconds(1);
    }

    return resData;
}

uint8_t Xn297::bitReverse(uint8_t d)
{
    uint8_t res = 0;

    for (int i = 0; i < 8; i++)
    {
        res = res << 1;
        res += (d & 0x01);
        d = d >> 1;
    }
    return res;
}

void Xn297::bitReverseEachElement(uint8_t *buf, uint8_t bufLen)
{
    for (int i = 0; i < bufLen; i++)
    {
        buf[i] = bitReverse(buf[i]);
    }
}

void Xn297::ceHigh()
{
    digitalWrite(ce, HIGH);
}

void Xn297::ceLow()
{
    digitalWrite(ce, LOW);
}

///////////////////////////////////////////////////////////////////////////////
// registers control :
///////////////////////////////////////////////////////////////////////////////

void Xn297::readRegister(uint8_t reg, uint8_t *dest, uint8_t len)
{
    digitalWrite(csn, LOW);
    spiTransfer(reg | regReadCmd);
    for (int i = 0; i < len; i++)
    {
        dest[i] = spiTransfer(0x00);
    }
    digitalWrite(csn, HIGH);

#ifdef XN297_DEBUG_OUT
    char debugBuf[128];
    if (len == 1)
    {
        sprintf(debugBuf, "%X -> %s", reg, bin(*dest, 8));
    }
    else
    {
        sprintf(debugBuf, "%X -> ", reg);
        for (int i = 0; i < len; i++)
        {
            sprintf(debugBuf + strlen(debugBuf), "%s ", hex(dest[i], 2));
        }
    }
    debug(debugBuf);
#endif
}

uint8_t Xn297::readRegister(uint8_t reg)
{
    uint8_t res;
    readRegister(reg, &res, 1);
    return res;
}

void Xn297::writeCommand(uint8_t cmd, uint8_t *data, uint8_t len)
{

    digitalWrite(csn, LOW);
    spiTransfer(cmd);
    for (int i = 0; i < len; i++)
    {
        spiTransfer(data[i]);
    }
    digitalWrite(csn, HIGH);
}

void Xn297::writeCommand(uint8_t cmd, uint8_t data)
{
    digitalWrite(csn, LOW);
    spiTransfer(cmd);
    spiTransfer(data);
    digitalWrite(csn, HIGH);
}

void Xn297::writeRegister(uint8_t reg, uint8_t *data, uint8_t len)
{
#ifdef XN297_DEBUG_OUT
    char debugBuf[128];
    sprintf(debugBuf, "%X <- ", reg);
    for (int i = 0; i < len; i++)
    {
        sprintf(debugBuf + strlen(debugBuf), "%s ", hex(data[i], 2));
    }
    debug(debugBuf);
#endif

    writeCommand(reg | regWriteCmd, data, len);
}

void Xn297::writeRegister(uint8_t reg, uint8_t data)
{
#ifdef XN297_DEBUG_OUT
    char debugBuf[128];
    sprintf(debugBuf, "%X <- %s", reg, bin(data, 8));
    debug(debugBuf);
#endif
    writeCommand(reg | regWriteCmd, data);
}

void Xn297::setBit(uint8_t reg, uint8_t b)
{
    uint8_t val = readRegister(reg);
    val = val | (1 << b);
    writeRegister(reg, val);
}

void Xn297::clearBit(uint8_t reg, uint8_t b)
{
    uint8_t val = readRegister(reg);
    val = val & (~(1 << b));
    writeRegister(reg, val);
}

uint8_t Xn297::readBit(uint8_t reg, uint8_t b)
{
    uint8_t val = readRegister(reg);
    return (val >> b) % 2;
}

///////////////////////////////////////////////////////////////////////////////
// settings:
///////////////////////////////////////////////////////////////////////////////
void Xn297::powerUp()
{
    setBit(0x00, 1);
}

void Xn297::shutDown()
{
    clearBit(0x00, 1);
}

void Xn297::setModeRX()
{
    digitalWrite(ce, LOW);
    // delay(1);
    setBit(0x00, 0);
    digitalWrite(ce, HIGH);
    // delay(1);
}

void Xn297::setRxPayloadWidth(uint8_t pipeNumber, uint8_t payloadW)
{
    pipeNumber = pipeNumber % 6;
    writeRegister(0x11 + pipeNumber, payloadW);
}

void Xn297::setCh(uint8_t ch)
{
    writeRegister(0x05, ch);
}

void Xn297::setDataRate(dataRate rate)
{

    if (rate == data_rate_2M)
    { // [b5 b3] = '01'
        setBit(0x06, 6);
        clearBit(0x06, 7);
    }
    else if (rate == data_rate_1M)
    { // [b5 b3] = '00'
        clearBit(0x06, 7);
        clearBit(0x06, 6);
    }
    else if (rate == data_rate_250k)
    {
        setBit(0x06, 7);
        setBit(0x06, 6);
    }
}

dataRate Xn297::getDataRate()
{
    uint8_t b6 = 0, b7 = 0;

    b6 = readBit(0x06, 6);
    b7 = readBit(0x06, 7);

    if (b6 == 1 && b7 == 0)
    {
        return data_rate_2M;
    }
    else if (b6 == 0 && b7 == 0)
    {
        return data_rate_1M;
    }
    else if (b6 == 1 && b7 == 1)
    {
        return data_rate_250k;
    }
    else
    {                        // '10' (should not occur)
        return data_rate_2M; // reserved / 2M
    }
}

void Xn297::setCRCSize(uint8_t crcSize)
{
    if (crcSize == 0)
    {
        clearBit(0x00, 3); // clear EN_CRC
    }
    else if (crcSize == 1)
    {
        // do nothing, these chip do not support 1-Byte CRC
    }
    else if (crcSize == 2)
    {
        setBit(0x00, 3); // set EN_CRC
    }
}

void Xn297::enableDynAck()
{
    setBit(0x1D, 0);
    // Enables the W_TX_PAYLOAD_NOACK (0xB0) command
}

void Xn297::disableDynAck()
{
    clearBit(0x1D, 0);
}

////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////
// RX:
////////////////////////////////////////////////////////////////////////////////
uint8_t Xn297::readRxPayloadWidth()
{ // of the last payload from RX FIFO
    uint8_t res;
    digitalWrite(csn, LOW);
    spiTransfer(0x60); // command R_RX_PL_WID
    res = spiTransfer(0x00);
    digitalWrite(csn, HIGH);
    return res;
}

void Xn297::readRxPayload(uint8_t *dest, uint8_t len)
{ // last from RX FIFO
    digitalWrite(csn, LOW);
    spiTransfer(0x61); // command R_RX_PAYLOAD
    for (int i = 0; i < len; i++)
    {
        dest[i] = spiTransfer(0x00);
    }
    digitalWrite(csn, HIGH);
}

////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////
// callbacks:
///////////////////////////////////////////////////////////////////////////////
void (Xn297::*onReceive)(void);

void Xn297::setOnReceive(void (*f)(void))
{
    onReceive = f;
}

void (Xn297::*onDebug)(const char *s);

void Xn297::dummyOnDebug(const char *s)
{
    // do nothing
}

void Xn297::setOnDebug(void (*f)(const char *s))
{
    onDebug = f;
}

void Xn297::debug(const char *s)
{
    onDebug(s);
}

///////////////////////////////////////////////////////////////////////////////

void Xn297::tick()
{

    if (readBit(0x07, 6))
    { // if DR triggered
        if (onReceive)
        {                // if function pointer was set
            onReceive(); // call it
        }
        // clear RX FIFO: (discard theremaining data, if any)
        /*uint8_t devnull;
        while( readBit(0x17, 0)==0 ) {
            readRxPayload((uint8_t*)(&devnull), 1);
        }*/
        setBit(0x07, 6); // clear DR (write 1 to clear)
    }
}

void Xn297::readSettings()
{
    char debugBuf[64];
    sprintf(debugBuf, "---------- settings ---------");
    debug(debugBuf);

    sprintf(debugBuf, "irq: %d, miso: %d, mosi: %d, sck: %d, csn: %d, ce: %d", irq, miso, mosi, sck, csn, ce);
    debug(debugBuf);

    readRegister(0x00);

    uint64_t buf = 0;
    readRegister(0x19, (uint8_t *)&buf, 1);
    buf = 0;

    readRegister(0x1E, (uint8_t *)&buf, 3);
    buf = 0;

    readRegister(0x1F, (uint8_t *)&buf, 5);
    buf = 0;

    readRegister(0x0A, (uint8_t *)&buf, 5);
    buf = 0;

    readRegister(0x0B, (uint8_t *)&buf, 5);
    buf = 0;

    sprintf(debugBuf, "---------- end ---------");
    debug(debugBuf);
}

void Xn297::begin(uint8_t irq, uint8_t miso, uint8_t mosi, uint8_t sck, uint8_t csn, uint8_t ce)
{
    usePins(irq, miso, mosi, sck, csn, ce);
    spiInit();
    writeRegister(0x00, 0x8E); // power on
    writeRegister(0x07, 0x70); // clear

    writeRegister(0x1D, 0x0); // CE controlled by pin

    uint8_t BB_CAL[] = {0x0A, 0x6D, 0x67, 0x9C, 0x46};
    writeRegister(0x1F, BB_CAL, 5);

    uint8_t RF_CAL[] = {0xF6, 0x37, 0x5D};
    writeRegister(0x1E, RF_CAL, 3);

    uint8_t DEMOD_CAL[] = {0x1};
    writeRegister(0x19, DEMOD_CAL, 1);

    uint8_t RF_CAL2[] = {0x45, 0x21, 0xEF, 0x2C, 0x5A, 0x40};
    writeRegister(0x1A, RF_CAL2, 6);

    uint8_t DEMOD_CAL2[] = {0x0B, 0xDF, 0x02};
    writeRegister(0x1B, DEMOD_CAL2, 3);

    writeRegister(0x01, 0x03); // auto ack
    writeRegister(0x02, 0x03); // data pipe enable
    writeRegister(0x03, 0x03); // addr width
    writeRegister(0x04, 0x02); // auto retransmit
    writeRegister(0x06, 0x3F); // data rate 1mbps

    writeRegister(0x11, 0x02); // payload length pipe 0
    writeRegister(0x12, 0x02); // payload length pipe 1
    writeRegister(0x1C, 0x00); // dynamic payload length disabled

    uint8_t ADDR1[] = {0xA7, 0x93, 0xB4, 0x55, 0xAA};
    uint8_t ADDR2[] = {0x81, 0xC6, 0xB2, 0xAA, 0x55};
    writeRegister(0x0A, ADDR1, 5);
    writeRegister(0x0B, ADDR2, 5);

    writeRegister(0x5, 0x31);  // channel 49 (dec)
    writeRegister(0x00, 0x8F); // RX en
    ceHigh();

    writeRegister(0x00, 0x8F); // RX en
#ifdef XN297_DEBUG_OUT
    readSettings();
#endif
}
