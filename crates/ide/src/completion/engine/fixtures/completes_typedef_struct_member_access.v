module top;
  typedef struct packed {
    logic [7:0] foo;
    logic bar;
  } my_t;

  my_t x;

  initial begin
    x./*caret*/
  end
endmodule
