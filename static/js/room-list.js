Vue.component('room-tr', { props: ['room'],
  template: '<tr><td>{{ room[0] }}</td><td>{{ room[1].player1.name }}</td><td v-if="room[1].player2 != null">{{ room[1].player2.name }}</td><td v-else></td></tr>', 
  computed: {
  }
})

let roomListV = new Vue({
  el: '#room-list',
  template: `
      <table rules="all">
        <tr>
          <th scope="col">Room Name</th>
          <th scope="col">Player1</th>
          <th scope="col">Player2</th>
        </tr>
        <tbody>
          <room-tr v-for="room in rooms"
            v-bind:room="room" 
            :key="room[0]"
          >
          </room-tr>
        </tbody>
      </table>`, 
  data() {
    return {
      rooms:[ 
      ],  
      conn: null, 
    };
  }, 
  methods: {
    listRooms: function() {
      this.conn.send("/listRooms");
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
      console.log(wsUri);
      that.conn = new WebSocket(wsUri);
      console.log('Connecting...');
      that.conn.onopen = function() {
        console.log('Connected.');
        that.listRooms();
      };
      that.conn.onmessage = function(e) {
        that.rooms = JSON.parse(e.data);
        console.log(JSON.parse(e.data));
        that.$forceUpdate();
      };
      that.conn.onclose = function() {
        console.log('Disconnected.');
        that.conn = null;
      };
    },
  }, 
})

roomListV.connect();
