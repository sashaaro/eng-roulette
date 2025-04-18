import {type RouteConfig, index, route} from "@react-router/dev/routes";

export default [
    index("routes/Home.tsx"),
    route("login", "routes/Login.tsx"),
    route("register", "routes/Register.tsx"),
    route("auth/google/callback", "routes/GoogleAuthCallback.tsx"),
] satisfies RouteConfig;
