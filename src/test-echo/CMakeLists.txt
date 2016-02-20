set(bdrck-test-echo_SOURCES

	bdrck-test-echo.cpp

)

add_executable(bdrck-test-echo ${bdrck-test-echo_SOURCES})
set_target_properties(bdrck-test-echo PROPERTIES RUNTIME_OUTPUT_DIRECTORY ${TESTS_OUTPUT_DIR})
target_link_libraries(bdrck-test-echo

	bdrck-params
	${bdrck_LIBRARIES}

)

include(${PROJECT_SOURCE_DIR}/3rdparty/cmu/cmake/IdeSplitSources.cmake)
cmuIdeSplitSources(bdrck-test-echo)
