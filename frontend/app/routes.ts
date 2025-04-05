import {type RouteConfig, index, route} from "@react-router/dev/routes";

export default [
    index("routes/Home.tsx"),
    route("login", "routes/Login.tsx"),
    route("register", "routes/register.tsx"),
] satisfies RouteConfig;
