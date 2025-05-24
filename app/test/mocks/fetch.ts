/**
 * It is easy to steal the cookie from this cookie jar.
 * Only use this cookie jar for testing purposes.
 */
class CookieJar {
	cookies: Map<string, string>;

	constructor() {
		this.cookies = new Map();
	}

	setCookie(cookieString: string) {
		const [nameValue] = cookieString.split(";");
		const [name, value] = nameValue.split("=");
		this.cookies.set(name.trim(), value.trim());
	}

	getCookieHeader() {
		return Array.from(this.cookies.entries())
			.map(([name, value]) => `${name}=${value}`)
			.join("; ");
	}
}

const cookieJar = new CookieJar();

// Grab a reference to the original Fetch API before it gets mocked.
// Otherwise, fetchWithCookies will recursively invoke itself indefinitely.
const originalFetch = global.fetch;

/**
 * A wrapper around the Node-based Fetch API that handles cookies
 * in a similar manner to how browsers handle cookies. It saves the
 * Set-Cookie headers and attaches them to subsequent requests.
 */
export const fetchWithCookies = async (
	url: string | URL | globalThis.Request,
	options?: RequestInit,
): Promise<Response> => {
	const headers = {
		...options?.headers,
		Cookie: cookieJar.getCookieHeader(),
	};

	const response = await originalFetch(url, { ...options, headers });
	const setCookieHeader = response.headers.get("Set-Cookie");

	if (setCookieHeader) {
		cookieJar.setCookie(setCookieHeader);
	}

	return response;
};
