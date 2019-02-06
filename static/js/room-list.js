Vue.component('room-tr', {
  props: ['name', 'standByPlayer', 'black', 'white'],
  template: '<tr><td>{{ name }}</td><td v-bind:class="classObject">{{ standByPlayer }}</td></tr>', 
  computed: {
    classObject: function() {
      return {
        'black-td': this.black, 
        'white-td': this.white, 
        'gray-td': !(this.black || this.white)
      }
    }
  }
})

let roomList = new Vue({
  el: '#room-list',
  template: `
      <table rules="all">
        <tbody>
          <room-tr v-for="room_name in Object.keys(rooms)"
            v-bind:name="room_name" 
            v-bind:standByPlayer="rooms[room_name].standByPlayer" 
            v-bind:black="rooms[room_name].black" 
            v-bind:white="rooms[room_name].white"
          >
          </room-tr>
        </tbody>
      </table>`, 
  data() {
    return {
      rooms: {
        "12345": {
          standByPlayer: "Hikota", 
          black: true, 
          white: null, 
        }, 
        "234524": {
          standByPlayer: "Hikota", 
          black: null, 
          white: null, 
        }
      }, 
      conn: null, 
    };
  }, 
  methods: {
    checkRoomStatus: function() {
      this.conn.send("/standByList");
    }, 
    disconnect: function() {
      if (this.conn != null) {
        console.log('Disconnecting...');
        this.conn.close();
        this.conn = null;
      }
    }, 
    connect: function() {
      let that = this;
      that.disconnect();
      var wsUri = (window.location.protocol=='https:'&&'wss://'||'ws://')+window.location.host + '/ws/';
      that.conn = new WebSocket(wsUri);
      console.log('Connecting...');
      that.conn.onopen = function() {
        console.log('Connected.');
      };
      that.conn.onmessage = function(e) {
        let message = JSON.parse(e.data);
        let mKind = message.kind;
        let mBody = message.body;
        console.log(message);
      };
      that.conn.onclose = function() {
        console.log('Disconnected.');
        that.conn = null;
      };
    },
  }, 
})

