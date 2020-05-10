import { Nes } from '../pkg';
import './index.scss';

const NES_FPS = 60;

class App {
	nes: Nes;
	ticker: number;
	nextFrame: number;

	constructor(nes: Nes) {
		this.nes = nes;
		this.ticker = -1;
		this.nextFrame = -1;

		this.render = this.render.bind(this);
		this.tickFrame = this.tickFrame.bind(this);
	}

	start(): void {
		this.ticker = window.setInterval(this.tickFrame, 1000 / NES_FPS);
		this.nextFrame = window.requestAnimationFrame(this.render);
	}

	render(): void {
		// TODO: Read Nes memory to get the image data?
		// Render image to a texture via WebGL or Canvas?

		// queue next frame
		this.nextFrame = window.requestAnimationFrame(this.render);
	}

	tickFrame(): void {
		try {
			this.nes.tick_frame();
		} catch {
			window.clearInterval(this.ticker);
			window.cancelAnimationFrame(this.nextFrame);
		}
	}
}

(async function() {
	const { Nes } = await import('../pkg');
	const app = new App(Nes.new());

	app.start();

	(window as any).app = app;
})();