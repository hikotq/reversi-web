Vue.config.devtools = false;
Vue.config.productionTip = false;

var vm = new Vue({
  el: '#app',
  data () {
    let board = [];
    for (var y = 0; y < 8; y++) {
      let row = []
      for (var x = 0; x < 8; x++) {
        row.push("empty");
      }
      board.push(row);
    }
    board[3][3] = "white";
    board[4][4] = "white";
    board[3][4] = "black";
    board[4][3] = "black";
    return {
      board: board,
      turn: null, 
      conn: null
    };
  },
  methods: {
    click_put: function(e) {
      let re = /index-(\d)(\d)/;
      let match = re.exec(e.currentTarget.className);
      let y = Number(match[1]);
      let x = Number(match[2]);
      console.log("x:" + x + ", y:" + y);
      console.log(this.canPut(y, x, "black"));
      if (this.canPut(x, y, "black")) {
        console.log("hi");
        this.put("black", x, y);
        this.send_move("Black", x, y);
      }
    }, 
    put: function(color, x, y) {
      this.board[y][x] = color;
      this.$forceUpdate();
    }, 
    canPut: function(x, y, turn) {
      let dir = [
          [1, 0],
          [1, 1],
          [0, 1],
          [-1, 1],
          [-1, 0],
          [-1, -1],
          [0, -1],
          [1, -1],
      ];
      let opposite = function(color) {
        if (color == "black") {
          return "white";
        }
        return "black";
      };
      let on_board = function(x, y) {
         return 0 <= x && x < 8 && 0 <= y && y < 8;
      };
      for(let d of dir) {
        let cx = x + d[0];
        let cy = y + d[1];
        if(on_board(cx, cy)) {
          if(this.board[cy][cx] != opposite(turn)) {
            continue;
          } 
          cx += d[0];
          cy += d[1];
        } else {
          continue;
        }
        while(on_board(cx, cy)) {
          if(this.board[cy][cx] == turn) {
            return true;
          } else if(this.board[cy][cx] == opposite(turn)) {
            break;
          }
          cx += d[0];
          cy += d[1];
        }
      }
      return false;
    }, 
    disconnect: function() {
      if (this.conn != null) {
        console.log('Disconnecting...');
        this.conn.close();
        this.conn = null;
      }
    }, 
    connect: function(callback) {
      let that = this;
      that.disconnect();
      var wsUri = (window.location.protocol=='https:'&&'wss://'||'ws://')+window.location.host + '/ws/';
      that.conn = new WebSocket(wsUri);
      console.log('Connecting...');
      that.conn.onopen = function() {
        console.log('Connected.');
        callback();
      };
      that.conn.onmessage = function(e) {
        console.log(e.data);
        let message = JSON.parse(e.data);
        if (message.kind == "Move") {
          x = message.body.Move.x;
          y = message.body.Move.y;
          color = message.body.Move.color.toLowerCase();
          that.put(color, x, y);
        }
        console.log(JSON.stringify(e.data))
      };
      that.conn.onclose = function() {
        console.log('Disconnected.');
        that.conn = null;
      };
    }, 
    send_move: function(color, x, y) {
      cmd = ["/move", color, x, y].join(' ');
      this.conn.send(cmd);
    }, 
    join(channel, uname) {
      let that = this;
      let cmd = ["/join", channel, uname].join(' ');
      this.connect(
        function() {
          that.conn.send(cmd);
        }
      );
    }, 
    makeRoom: function(channel, uname, color) {
      let that = this;
      let cmd_array = ["/makeRoom", channel, uname];
      if(color != null) {
        that.color
        cmd_array.push(color);
      }
      let cmd = cmd_array.join(' ');
      this.connect(
        function() {
          that.conn.send(cmd);
        }
      );
    }, 
  }
});
