import './index.scss';

(async function() {
	const { Test } = await import('../pkg');

	(window as any).Test = Test;
})();