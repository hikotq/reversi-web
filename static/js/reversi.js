var EMPTY = "empty";
var WHITE = "white";
var BLACK = "black";
var AVAILABLE = "available";

Vue.config.devtools = true;
Vue.config.productionTip = true;


Vue.component('board', {
  props: ['board'], 
  template: `
    <table class="board">
      <tr class="board_row" v-for="(row, idx1) in board">
        <td class="board_col" v-bind:class="'index-'+idx1+idx2" v-for="(piece, idx2) in row" v-on:click="clickCell">
          <div class="content">
            <span v-bind:class="piece+'-piece'"></span>
          </div>
        </td>
      </tr>
    </table>`, 
  methods: {
    clickCell: function(e) {
      this.$emit('click-cell', e);
    }, 
  }
})


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
    board[3][3] = WHITE;
    board[4][4] = WHITE;
    board[3][4] = BLACK;
    board[4][3] = BLACK;
    board[3][2] = AVAILABLE;
    board[2][3] = AVAILABLE;
    board[4][5] = AVAILABLE;
    board[5][4] = AVAILABLE;
    return {
      board: board,
      turn: null, 
      ownColor: null, 
      conn: null, 
    };
  },
  methods: {
    put: function(e) {
      console.log("click_put!");
      let re = /index-(\d)(\d)/;
      let match = re.exec(e.currentTarget.className);
      let y = Number(match[1]);
      let x = Number(match[2]);
      console.log("x:" + x + ", y:" + y);
      console.log(this.canPut(y, x, this.turn));
      if (this.conn != null && this.turn == this.ownColor && this.canPut(x, y, this.turn)) {
        this.send_move(this.turn, x, y);
      }
    }, 
    canPut: function(x, y) {
      return this.board[y][x] == AVAILABLE;
    }, 
    oppositeColor: function(color) {
      if(color.toLowerCase() == 'black') {
        return WHITE;
      }    
      return BLACK;
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
        let mKind = message.kind;
        let mBody = message.body;
        let board = [];
        switch(mKind) {
          case 'GameStart':
            let color = mBody.GameStart.toLowerCase();
            that.turn = BLACK;
            that.ownColor = color;
            break;
          case 'Game':
            board = mBody.Game.board;
            that.turn = mBody.Game.turn.toLowerCase();
            for (var y = 0; y < 8; y++) {
              for (var x = 0; x < 8; x++) {
                that.board[y].splice(x, 1, board[y * 8 + x]);
              }
            }
            break;
          case 'GameOver':
            let game = mBody.GameOver[0];
            board = game.board;
            let winner = mBody.GameOver[1];
            that.turn = game.turn.toLowerCase();
            for (var y = 0; y < 8; y++) {
              for (var x = 0; x < 8; x++) {
                that.board[y].splice(x, 1, board[y * 8 + x]);
              }
            }
            swal("Game is over!", winner + " is  winner!");
          default:
            break;
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
        that.color = color;
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
