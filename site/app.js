// table
var entries = new Vue({
  el: '#entries',
  created: function() {
    this.$cookies.set('one', 1);
    console.log(this.$cookies.get('one'));
  },
  data: {
    items: [{
      isActive: true,
      age: 40,
      name: {
        first: 'Dickerson',
        last: 'Macdonald'
      }
    }, {
      isActive: false,
      age: 21,
      name: {
        first: 'Larsen',
        last: 'Shaw'
      }
    }, {
      isActive: false,
      age: 9,
      state: 'success',
      name: {
        first: 'Mitzi',
        last: 'Navarro'
      }
    }, {
      isActive: false,
      age: 89,
      name: {
        first: 'Geneva',
        last: 'Wilson'
      }
    }, {
      isActive: true,
      age: 38,
      name: {
        first: 'Jami',
        last: 'Carney'
      }
    }, {
      isActive: false,
      age: 27,
      name: {
        first: 'Essie',
        last: 'Dunlap'
      }
    }, {
      isActive: true,
      age: 40,
      name: {
        first: 'Dickerson',
        last: 'Macdonald'
      }
    }, {
      isActive: false,
      age: 21,
      name: {
        first: 'Larsen',
        last: 'Shaw'
      }
    }, {
      isActive: false,
      age: 26,
      name: {
        first: 'Mitzi',
        last: 'Navarro'
      }
    }, {
      isActive: false,
      age: 22,
      name: {
        first: 'Geneva',
        last: 'Wilson'
      }
    }, {
      isActive: true,
      age: 38,
      name: {
        first: 'Jami',
        last: 'Carney'
      }
    }, {
      isActive: false,
      age: 27,
      name: {
        first: 'Essie',
        last: 'Dunlap'
      }
    }],
    fields: {
      name: {
        label: 'Person Full name',
        sortable: true
      },
      age: {
        label: 'Person age',
        sortable: true
      },
      isActive: {
        label: 'Is Active',
        sortable: true
      },
      actions: {
        label: 'Actions'
      }
    },
    currentPage: 1,
    perPage: 5,
    filter: null
  },
  methods: {
    details(item) {
      alert(JSON.stringify(item));
    }
  }
})

// datetimepicker
var dt = new Vue({
  el: '#dt',
  data: {
    pickerOptions1: {
      shortcuts: [{
        text: 'Today',
        onClick(picker) {
          picker.$emit('pick', new Date());
        }
      }, {
        text: 'Yesterday',
        onClick(picker) {
          const date = new Date();
          date.setTime(date.getTime() - 3600 * 1000 * 24);
          picker.$emit('pick', date);
        }
      }, {
        text: 'A week ago',
        onClick(picker) {
          const date = new Date();
          date.setTime(date.getTime() - 3600 * 1000 * 24 * 7);
          picker.$emit('pick', date);
        }
      }]
    },
    dtValue: ''
  },
});