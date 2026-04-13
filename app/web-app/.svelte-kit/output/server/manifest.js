export const manifest = (() => {
function __memo(fn) {
	let value;
	return () => value ??= (value = fn());
}

return {
	appDir: "_app",
	appPath: "_app",
	assets: new Set([]),
	mimeTypes: {},
	_: {
		client: {start:"_app/immutable/entry/start.DPCWiHyQ.js",app:"_app/immutable/entry/app.ClUIzUwI.js",imports:["_app/immutable/entry/start.DPCWiHyQ.js","_app/immutable/chunks/DYn6Fzor.js","_app/immutable/chunks/DMwxfA9P.js","_app/immutable/chunks/CWeFt6jb.js","_app/immutable/chunks/C2r77BOQ.js","_app/immutable/entry/app.ClUIzUwI.js","_app/immutable/chunks/DMwxfA9P.js","_app/immutable/chunks/Cjriaz1E.js","_app/immutable/chunks/C2r77BOQ.js","_app/immutable/chunks/By017JCv.js","_app/immutable/chunks/vBSHuzwv.js","_app/immutable/chunks/T9zTCS3s.js"],stylesheets:[],fonts:[],uses_env_dynamic_public:false},
		nodes: [
			__memo(() => import('./nodes/0.js')),
			__memo(() => import('./nodes/1.js')),
			__memo(() => import('./nodes/2.js')),
			__memo(() => import('./nodes/3.js')),
			__memo(() => import('./nodes/4.js')),
			__memo(() => import('./nodes/5.js')),
			__memo(() => import('./nodes/6.js')),
			__memo(() => import('./nodes/7.js')),
			__memo(() => import('./nodes/8.js')),
			__memo(() => import('./nodes/9.js'))
		],
		remotes: {
			
		},
		routes: [
			{
				id: "/",
				pattern: /^\/$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 2 },
				endpoint: null
			},
			{
				id: "/dashboard",
				pattern: /^\/dashboard\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 3 },
				endpoint: null
			},
			{
				id: "/executions",
				pattern: /^\/executions\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 4 },
				endpoint: null
			},
			{
				id: "/hypercubes",
				pattern: /^\/hypercubes\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 5 },
				endpoint: null
			},
			{
				id: "/portfolios",
				pattern: /^\/portfolios\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 6 },
				endpoint: null
			},
			{
				id: "/rate-matrices",
				pattern: /^\/rate-matrices\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 7 },
				endpoint: null
			},
			{
				id: "/studies",
				pattern: /^\/studies\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 8 },
				endpoint: null
			},
			{
				id: "/study-units",
				pattern: /^\/study-units\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 9 },
				endpoint: null
			}
		],
		prerendered_routes: new Set([]),
		matchers: async () => {
			
			return {  };
		},
		server_assets: {}
	}
}
})();
