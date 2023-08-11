<template>
  <div class="w-full flex flex-col h-full items-center">
    <div class="bg-black w-[27rem] lg:w-[53rem] 2xl:w-[79rem] h-0.5 mt-4"></div>
    <div class="overflow-y-scroll overflow-hidden block h-full scrollbar-hide">
      <div class="grid grid-cols-1 lg:grid-cols-2 2xl:grid-cols-3 gap-8 p-4">
        <AdminPlacePost v-for="place in places" :key="place.place_id" :name="place.name" online/>
        <div class="bg-white border-2 border-black rounded-2xl w-[24rem] h-[24rem] p-4 flex flex-col shadow-custom order-first">
          <div class="flex flex-col items-center">
            <p class="text-xl">New Place</p>
            <div class="bg-black w-3/5 h-0.5 mt-1"></div>
          </div>
          <p class="mt-2">Name :</p>
          <input v-model="name" type="text" class="border-2 border-black rounded-lg w-full px-2"/>
          <p class="mt-2">Width :</p>
          <input v-model="size" type="number" class="border-2 border-black rounded-lg w-full px-2" min="8" max="512" step="8"/>
          <p class="mt-2">Cooldown :</p>
          <input v-model="cooldown" type="number" class="border-2 border-black rounded-lg w-full px-2" min="1" max="3600"/>
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
let cooldown = ref(60);

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
    "palette": palette.value,
    "cooldown": cooldown.value
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
    await getPlaces();
  }
}
</script>