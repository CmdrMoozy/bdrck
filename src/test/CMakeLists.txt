set(bdrck-test_SOURCES

	bdrck-test.cpp

	algorithm/StringTest.cpp

	fs/TemporaryStorageTest.cpp

	params/CommandTest.cpp
	params/OptionTest.cpp

	params/detail/ParseArgumentsTest.cpp
	params/detail/ParseCommandTest.cpp
	params/detail/ParseOptionsTest.cpp

	util/ScopeExitTest.cpp

)

add_executable(bdrck-test ${bdrck-test_SOURCES})
target_link_libraries(bdrck-test
	bdrck-algorithm
	bdrck-cwrap
	bdrck-fs
	bdrck-params
	bdrck-process
	bdrck-util
)
add_test(bdrck-test bdrck-test)
