const LATEST_RELEASE = 'https://api.github.com/repos/DeisDev/nwmpublisher/releases/latest';
const STABLE_VERSION = /^v?(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/;

function versionParts(version) {
	if (typeof version !== 'string') return null;
	const match = version.match(STABLE_VERSION);
	if (!match) return null;
	const parts = match.slice(1, 4).map(Number);
	return parts.every(Number.isSafeInteger) ? parts : null;
}

export function isNewerVersion(current, candidate) {
	const currentParts = versionParts(current);
	const candidateParts = versionParts(candidate);
	if (!currentParts || !candidateParts) return false;

	for (let i = 0; i < 3; i++) {
		if (currentParts[i] !== candidateParts[i]) {
			return candidateParts[i] > currentParts[i];
		}
	}
	return false;
}

// The latest-release endpoint supplies stable releases; failed checks leave the UI unchanged.
export async function getUpdateVersion(current, fetchRelease = globalThis.fetch) {
	try {
		const response = await fetchRelease(LATEST_RELEASE);
		if (!response.ok) return null;
		const release = await response.json();
		if (!release || release.draft || release.prerelease) return null;
		return isNewerVersion(current, release.tag_name) ? release.tag_name : null;
	} catch {
		return null;
	}
}
