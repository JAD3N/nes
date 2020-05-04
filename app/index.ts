import { Nes } from '../pkg';
import './index.scss';

const NES_FPS = 60;

class App {
	nes: Nes;
	clocker: number;
	nextFrame: number;

	constructor(nes: Nes) {
		this.nes = nes;
		this.clocker = -1;
		this.nextFrame = -1;

		this.render = this.render.bind(this);
		this.clockFrame = this.clockFrame.bind(this);
	}

	start(): void {
		this.clocker = window.setInterval(this.clockFrame, 1000 / NES_FPS);
		this.nextFrame = window.requestAnimationFrame(this.render);
	}

	render(): void {
		// TODO: Read Nes memory to get the image data?
		// Render image to a texture via WebGL or Canvas?

		// queue next frame
		this.nextFrame = window.requestAnimationFrame(this.render);
	}

	clockFrame(): void {
		this.nes.tick_frame();
	}
}

(async function() {
	const { Nes } = await import('../pkg');
	const app = new App(Nes.new());

	app.start();

	(window as any).app = app;
})();