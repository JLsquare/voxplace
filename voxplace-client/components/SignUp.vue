<template>
  <div class="bg-white border-2 border-black rounded-2xl w-[28rem] flex flex-col items-center p-8 shadow-custom relative">
    <CloseButton class="absolute top-4 right-4" @click="$emit('close-clicked')" />
    <p class="text-3xl">Sign Up</p>
    <div class="bg-black w-3/5 h-0.5 mt-2"></div>
    <div class="bg-white border-2 border-black rounded-2xl w-96 mt-8 flex justify-between items-center px-4 shadow-custom">
      <input v-model="username" type="text" placeholder="Username" class="text-xl text-neutral-500 mb-1 bg-white w-full outline-none" />
    </div>
    <div class="mt-2 h-6 flex items-start w-full">
      <p v-if="invalidUsername" class="text-red-500 text-sm">Username must be at least 3 characters long.</p>
    </div>
    <div class="bg-white border-2 border-black rounded-2xl w-96 mt-2 flex justify-between items-center px-4 shadow-custom">
      <input v-model="email" type="email" placeholder="Email" class="text-xl text-neutral-500 mb-1 bg-white w-full outline-none" />
    </div>
    <div class="mt-2 h-6 flex items-start w-full">
      <p v-if="invalidEmail" class="text-red-500 text-sm">Email must be valid.</p>
    </div>
    <div class="bg-white border-2 border-black rounded-2xl w-96 mt-2 flex justify-between items-center px-4 shadow-custom">
      <input v-model="password" type="password" placeholder="Password" class="text-xl text-neutral-500 mb-1 bg-white w-full outline-none" />
    </div>
    <div class="mt-2 h-6 flex items-start w-full">
      <p v-if="invalidPassword" class="text-red-500 text-sm">Password must be at least 8 characters long.</p>
    </div>
    <Button @click="signup" text="Sign Up" class="mt-2 bg-blue-300 hover:bg-blue-400"/>
    <div class="flex mt-8 items-start w-full">
      <p>Already have an account?</p>
      <p @click="$emit('login-clicked')" class="underline text-blue-400 ml-2 cursor-pointer">Log In</p>
    </div>
  </div>
</template>

<script setup>
let username = ref('');
let email = ref('');
let password = ref('');
let token = ref('');

let invalidUsername = ref(false);
let invalidEmail = ref(false);
let invalidPassword = ref(false);

const emailRegex = /^[\w-]+(\.[\w-]+)*@([\w-]+\.)+[a-zA-Z]{2,7}$/;
const emit = defineEmits(['login-clicked', 'close-clicked', 'signed-up']);

async function signup() {
  invalidUsername.value = false;
  invalidEmail.value = false;
  invalidPassword.value = false;

  invalidUsername.value = username.value.trim().length < 3;
  invalidEmail.value = !emailRegex.test(email.value.trim());
  invalidPassword.value = password.value.trim().length < 8;

  if (invalidUsername.value || invalidEmail.value || invalidPassword.value) {
    return;
  }

  const response = await fetch(`http://${window.location.hostname}:8000/api/user/register`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      username: username.value,
      email: email.value,
      password: password.value
    })
  });

  if (!response.ok) {
    const message = await response.text();
    throw new Error(message);
  }

  token.value = await response.json();

  localStorage.setItem('token', token.value);

  emit('signed-up');
}
</script>