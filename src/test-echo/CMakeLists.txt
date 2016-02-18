set(bdrck-test-echo_SOURCES

	bdrck-test-echo.cpp

)

add_executable(bdrck-test-echo ${bdrck-test-echo_SOURCES})
target_link_libraries(bdrck-test-echo

	bdrck-params
	${bdrck_LIBRARIES}

)
