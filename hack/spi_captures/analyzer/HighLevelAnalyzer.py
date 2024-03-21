# High Level Analyzer
# For more information and documentation, please go to https://support.saleae.com/extensions/high-level-analyzer-extensions

from saleae.analyzers import HighLevelAnalyzer, AnalyzerFrame, StringSetting, NumberSetting, ChoicesSetting
import enum

class PrevFrame(enum.Enum):
    START = 1
    OPCODE = 2
    DATA = 3
    STOP = 4

OPCODES_MAPPING = {
    "000": "R_REGISTER",  # + 5 bit addr
    "001": "W_REGISTER",  # + 5 bit addr
    "10101": "W_ACK_PAYLOAD",  # + 3 bit amount
    "01100001": "R_RX_PAYLOAD",
    "10100000": "W_TX_PAYLOAD",
    "11100001": "FLUSH_TX",
    "11100010": "FLUSH_RX",
    "11100011": "REUSE_TX_PL",
    "01010000": "ACTIVATE/DEACTIVATE",
    "01100000": "R_RX_PL_WID",
    "10110000": "W_TX_PAYLOAD_NOACK",
    "11111101": "CE_FSPI_ON",
    "11111100": "CE_FSPI_OFF",
    "01010011": "RST_FSPI_HOLD/RELS",
    "11111111": "NOP",
}


def rx_addr_handler(data_accumulator):
    barr = []
    for _byte in data_accumulator:
        barr.append(int(_byte, 2).to_bytes(1, "big").hex().upper())

    return '0x' + ''.join(barr)


def radio_ch_handler(data_accumulator):
    data = data_accumulator[0][1:]
    if data == '1001110':
        return 'RESET'
    return ' '.join(data_accumulator)


def noop_handler(data_accumulator):
    return ' '.join(data_accumulator)


REGISTER_DATA_HANDLER = {
    '0xa': rx_addr_handler,
    '0xb': rx_addr_handler,

    '0x5': radio_ch_handler,
    '0x7': noop_handler,
}


def get_opcode(binary_word):
    rw_preambles = ["000", "001"]
    ack_preamble = "10101"
    opcode = binary_word
    preamble, data = str(binary_word)[:3], str(binary_word)[3:]
    if preamble in rw_preambles:
        opcode = preamble
        hr_opcode = OPCODES_MAPPING.get(opcode)
        hr_data = hex(int(data, 2))

        handler = REGISTER_DATA_HANDLER.get(hr_data, noop_handler)
        return f"{hr_opcode} {hr_data}", handler

    ack_p, data = str(binary_word)[:5], str(binary_word)[5:]
    if ack_p == ack_preamble:
        opcode = ack_p
        hr_opcode = OPCODES_MAPPING.get(opcode)
        hr_data = int(data, 2)
        return f"{hr_opcode} {hr_data}", noop_handler

    return OPCODES_MAPPING.get(opcode, "UNKNOWN"), noop_handler


# High level analyzers must subclass the HighLevelAnalyzer class.
class Hla(HighLevelAnalyzer):
    # List of settings that a user can set for this High Level Analyzer.
    # my_string_setting = StringSetting()
    # my_number_setting = NumberSetting(min_value=0, max_value=100)
    # my_choices_setting = ChoicesSetting(choices=('A', 'B'))

    # An optional list of types this analyzer produces, providing a way to customize the way frames are displayed in Logic 2.
    # result_types = {
    #     'mytype': {
    #         'format': 'Output type: {{type}}, Input type: {{data.input_type}}'
    #     }
    # }
    result_types = {
        'command': {
            'format': '{{data.opcode}}, Arg: {{data.data}}'
        },
    }

    def __init__(self, *args, **kwargs):
        '''
        Initialize HLA.

        Settings can be accessed using the same name used above.
        '''

        self.prev_frame = None
        self.start_time = None
        self.end_time = None

        self.opcode = None
        self.data_accumulator = []

        print('init')

    @staticmethod
    def to_bitarray(byte):
        return bin(int.from_bytes(byte, "big"))[2:].zfill(8)

    def decode(self, frame: AnalyzerFrame):
        '''
        Process a frame from the input analyzer, and optionally return a single `AnalyzerFrame` or a list of `AnalyzerFrame`s.

        The type and data values in `frame` will depend on the input analyzer.
        '''
        ftype = frame.type

        if ftype == 'error':
            self.prev_frame = None
            self.start_time = None
            self.end_time = None
            self.data_accumulator = []
            self.opcode = None
            return

        if ftype == 'enable':
            self.data_accumulator = []
            self.prev_frame = PrevFrame.START
            self.start_time = frame.start_time
            return

        mosi = frame.data.get('mosi', '')

        if ftype == 'result' and self.prev_frame == PrevFrame.START:
            self.prev_frame = PrevFrame.OPCODE
            bmosi = self.to_bitarray(mosi)
            get_opcode(bmosi)
            self.opcode = bmosi
            return

        if ftype == 'result' and self.prev_frame == PrevFrame.OPCODE:
            bmosi = self.to_bitarray(mosi)
            self.data_accumulator.append(bmosi)
            return

        if ftype == 'disable':
            self.prev_frame = PrevFrame.STOP
            self.end_time = frame.end_time

            opcode, handler = get_opcode(self.opcode)
            data = handler(self.data_accumulator) if handler else ' '.join(self.data_accumulator)

            return AnalyzerFrame('command', self.start_time, self.end_time, {
                'opcode': opcode,
                'data': data,
            })
