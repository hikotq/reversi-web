Vue.component('room-td', {
  props: ['name', 'standByPlayer', 'black', 'white'],
  template: '<div><td>{{ name }}</td><td v-bind:class="classObject">{{ standByPlayer }}</td></div>', 
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

new Vue({
  el: '#room-list',
  template: `
      <tbody>
        <tr v-for="room_name in Object.keys(rooms)">
          <room-td
          v-bind:name="room_name" 
          v-bind:standByPlayer="rooms[room_name].standByPlayer" 
          v-bind:black="rooms[room_name].black" 
          v-bind:white="rooms[room_name].white"
          ></room-td>
        </tr>
      </tbody>`, 
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
        let room = JSON.parse(e.data);
        this.rooms[name] = {
          standByPlayer: room.stand_by_player, 
          black: room.black, 
          white: room.white, 
        };
        this.$forceUpdate();
      };
      that.conn.onclose = function() {
        console.log('Disconnected.');
        that.conn = null;
      };
    },
  }, 
})

