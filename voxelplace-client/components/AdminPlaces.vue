<template>
  <div class="w-full flex flex-col h-full items-center">
    <div class="bg-black w-[33rem] lg:w-[65rem] 2xl:w-[97rem] h-0.5 mt-4"></div>
    <div class="overflow-y-scroll overflow-hidden block h-full scrollbar-hide">
      <div class="grid grid-cols-2 lg:grid-cols-3 2xl:grid-cols-4 gap-8 p-4">
        <AdminPlacePost v-for="place in places" :key="place.id" :name="place" online/>
        <div class="bg-white border-2 border-black rounded-2xl w-[20rem] h-[20rem] p-4 flex flex-col shadow-custom order-first">
          <div class="flex flex-col items-center">
            <p class="text-xl">New Place</p>
            <div class="bg-black w-3/5 h-0.5 mt-1"></div>
          </div>
          <p class="mt-2">Name :</p>
          <input v-model="name" type="text" class="border-2 border-black rounded-lg w-full px-2"/>
          <p class="mt-2">Width :</p>
          <input v-model="size" type="number" class="border-2 border-black rounded-lg w-full px-2" min="8" max="512" step="8"/>
          <p class="mt-2">Palette :</p>
          <select v-model="palette" class="bg-white border-2 border-black rounded-lg w-full px-2">
            <option value="1">r/place</option>
          </select>
          <div class="flex justify-center mt-8">
            <Button text="Create" class="bg-blue-300" @click="createPlace"/>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
let places = ref([]);
let name = ref('');
let size = ref(128);
let palette = ref('');

onMounted(() => {
  getPlaces();
});

async function getPlaces() {
  const res = await fetch(`http://${window.location.hostname}:8000/api/place/infos`);
  const data = await res.json();
  console.log(data);
  places.value = data;
}

async function createPlace() {
  let createPlaceRequest = {
    "name": name.value,
    "size": [size.value, size.value, size.value],
    "palette": palette.value
  };

  const response = await fetch(`http://${window.location.hostname}:8000/api/place/create`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': localStorage.getItem('token')
    },
    body: JSON.stringify(createPlaceRequest)
  });

  if (!response.ok) {
    const message = await response.text();
    console.log(message);
  } else {
    getPlaces();
  }
}
</script>