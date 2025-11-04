typedef struct packed {
    logic [7:0] data;
} packet_t;

module test;
    packet_t existing;
    int packet_count;

    initial begin
        pa$0 next_value;
    end
endmodule
