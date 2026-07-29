const LOADING_TEXT_ID = "loading_text";
const LOADING_ID = "loading";

export default function myInitializer() {
	return {
		onStart: () => {
			document.getElementById(LOADING_TEXT_ID).textContent = "Loading WASM...";
		},
		onProgress: ({ current, total }) => {
			document.getElementById(LOADING_TEXT_ID).textContent = `Loading WASM...\n${Math.round((current / total) * 100)}%`;
		},
		onComplete: () => {

		},
		onSuccess: (wasm) => {
			document.getElementById(LOADING_ID).remove();
		},
		onFailure: (error) => {
			document.getElementById(LOADING_TEXT_ID).textContent = "Failed to load WASM:\n" + error;
		},
	};
}
