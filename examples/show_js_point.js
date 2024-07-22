function show_point(point) {
    if (!point) {
        console.log("point is null");
        return;
    }

    console.log("point.x =", point.x, "point.y = ", point.y, "point.multiple() = ", point.multiple());

    point.x = point.x + 1;
    point.y = point.x + 2;

    return point;
}